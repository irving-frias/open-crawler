//! Native desktop share sheet.
//!
//! On macOS this opens the system share picker (`NSSharingServicePicker`) for a
//! file path, which offers AirDrop and other Bluetooth/WiFi-based transfer
//! services. Other desktop platforms have no universal share dialog, so they
//! fall back to a friendly error.

use tauri::AppHandle;

#[cfg(target_os = "macos")]
mod macos {
    use objc2::rc::{autoreleasepool, Retained};
    use objc2::runtime::{AnyObject, ProtocolObject};
    use objc2::{define_class, msg_send, AnyThread, DefinedClass, MainThreadOnly, Message};
    use objc2_app_kit::{
        NSSharingService, NSSharingServiceDelegate, NSSharingServicePicker,
        NSSharingServicePickerDelegate, NSView,
    };
    use objc2_core_foundation::{CGPoint, CGRect, CGSize};
    use objc2_foundation::{
        MainThreadMarker, NSArray, NSError, NSObject, NSObjectProtocol, NSString, NSURL,
    };
    use raw_window_handle::{HasWindowHandle, RawWindowHandle, WindowHandle};
    use std::cell::RefCell;
    use std::sync::mpsc;
    use std::time::Duration;
    use tauri::{AppHandle, Manager, WebviewWindow};

    const SHARE_COMPLETION_TIMEOUT: Duration = Duration::from_secs(300);

    thread_local! {
        static ACTIVE_DELEGATES: RefCell<Vec<Retained<SharePickerDelegate>>> = const { RefCell::new(Vec::new()) };
    }

    #[derive(Default)]
    struct ShareDelegateIvars {
        completion: RefCell<Option<mpsc::Sender<Result<(), String>>>>,
    }

    define_class!(
        #[unsafe(super = NSObject)]
        #[thread_kind = MainThreadOnly]
        #[ivars = ShareDelegateIvars]
        struct SharePickerDelegate;

        unsafe impl NSObjectProtocol for SharePickerDelegate {}

        unsafe impl NSSharingServicePickerDelegate for SharePickerDelegate {
            #[unsafe(method_id(sharingServicePicker:delegateForSharingService:))]
            fn sharing_service_picker_delegate_for_sharing_service(
                &self,
                _picker: &NSSharingServicePicker,
                _service: &NSSharingService,
            ) -> Option<Retained<ProtocolObject<dyn NSSharingServiceDelegate>>> {
                Some(ProtocolObject::from_retained(self.retain()))
            }

            #[unsafe(method(sharingServicePicker:didChooseSharingService:))]
            fn sharing_service_picker_did_choose_sharing_service(
                &self,
                _picker: &NSSharingServicePicker,
                service: Option<&NSSharingService>,
            ) {
                if service.is_none() {
                    self.complete(Ok(()));
                }
            }
        }

        unsafe impl NSSharingServiceDelegate for SharePickerDelegate {
            #[unsafe(method(sharingService:didShareItems:))]
            fn sharing_service_did_share_items(
                &self,
                _service: &NSSharingService,
                _items: &NSArray,
            ) {
                self.complete(Ok(()));
            }

            #[unsafe(method(sharingService:didFailToShareItems:error:))]
            fn sharing_service_did_fail_to_share_items_error(
                &self,
                _service: &NSSharingService,
                _items: &NSArray,
                error: &NSError,
            ) {
                let message = autoreleasepool(|pool| unsafe {
                    error.localizedDescription().to_str(pool).to_string()
                });
                self.complete(Err(format!("Sharing failed: {message}")));
            }
        }
    );

    impl SharePickerDelegate {
        fn new(
            mtm: MainThreadMarker,
            completion: mpsc::Sender<Result<(), String>>,
        ) -> Retained<Self> {
            let ivars = ShareDelegateIvars {
                completion: RefCell::new(Some(completion)),
            };
            let this = Self::alloc(mtm).set_ivars(ivars);
            unsafe { msg_send![super(this), init] }
        }

        fn complete(&self, result: Result<(), String>) {
            if let Some(tx) = self.ivars().completion.borrow_mut().take() {
                let _ = tx.send(result);
            }
            let ptr = self as *const SharePickerDelegate;
            ACTIVE_DELEGATES.with(|delegates| {
                let mut list = delegates.borrow_mut();
                if let Some(pos) = list.iter().position(|item| Retained::as_ptr(item) == ptr) {
                    list.remove(pos);
                }
            });
        }
    }

    /// Shares a single file through the native macOS share picker (AirDrop, …).
    ///
    /// Blocks until the picker is dismissed (or `SHARE_COMPLETION_TIMEOUT`
    /// elapses) so the caller knows whether the user actually shared.
    pub fn share_file<R: tauri::Runtime>(app: &AppHandle<R>, path: &str) -> Result<(), String> {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| "No main window available".to_string())?;
        let (setup_tx, setup_rx) = mpsc::channel();
        let (completion_tx, completion_rx) = mpsc::channel();
        let window_clone = window.clone();
        let path = path.to_string();

        window
            .run_on_main_thread(move || {
                let result = (|| -> Result<(), String> {
                    let ns_view = get_ns_view(&window_clone)?;
                    let file_url = NSURL::fileURLWithPath(&NSString::from_str(&path));
                    let item: Retained<NSObject> = unsafe { Retained::cast_unchecked(file_url) };
                    let object_ref: &AnyObject = item.as_ref() as &AnyObject;

                    let share_result: Result<(), String> = autoreleasepool(|_pool| {
                        let items_array = NSArray::from_slice(&[object_ref]);
                        let picker = unsafe {
                            NSSharingServicePicker::initWithItems(
                                NSSharingServicePicker::alloc(),
                                &items_array,
                            )
                        };

                        let mtm = MainThreadMarker::new()
                            .ok_or_else(|| "Main thread marker unavailable".to_string())?;
                        let delegate = SharePickerDelegate::new(mtm, completion_tx);
                        ACTIVE_DELEGATES
                            .with(|delegates| delegates.borrow_mut().push(delegate.retain()));
                        picker.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

                        let bounds = ns_view.bounds();
                        picker.showRelativeToRect_ofView_preferredEdge(
                            CGRect {
                                origin: CGPoint {
                                    x: bounds.size.width / 2.0,
                                    y: bounds.size.height / 2.0,
                                },
                                size: CGSize {
                                    width: 0.0,
                                    height: 0.0,
                                },
                            },
                            &ns_view,
                            objc2_foundation::NSRectEdge::NSMinYEdge,
                        );
                        Ok(())
                    });
                    share_result?;
                    Ok(())
                })();
                let _ = setup_tx.send(result);
            })
            .map_err(|e| e.to_string())?;

        setup_rx.recv().map_err(|e| e.to_string())??;

        match completion_rx.recv_timeout(SHARE_COMPLETION_TIMEOUT) {
            Ok(result) => result,
            Err(mpsc::RecvTimeoutError::Timeout) => Ok(()),
            Err(mpsc::RecvTimeoutError::Disconnected) => Ok(()),
        }
    }

    /// Retrieves the native `NSView` of the Tauri window, compatible with
    /// `raw-window-handle`.
    fn get_ns_view<R: tauri::Runtime>(
        window: &WebviewWindow<R>,
    ) -> Result<Retained<NSView>, String> {
        let window_handle: WindowHandle<'_> = window.window_handle().map_err(|e| e.to_string())?;
        if let RawWindowHandle::AppKit(handle) = window_handle.as_raw() {
            let ns_view_ptr = handle.ns_view.as_ptr();
            unsafe { Retained::retain(ns_view_ptr.cast()) }
                .ok_or_else(|| "Failed to retain NSView".to_string())
        } else {
            Err("Unsupported window handle type on macOS".to_string())
        }
    }
}

/// Opens the native share sheet for `path`, if the platform supports one.
pub fn share_file(app: &AppHandle, path: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        macos::share_file(app, path)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, path);
        Err(
            "No native share sheet is available on this platform. Use WiFi sharing or the Export tab instead."
                .to_string(),
        )
    }
}
