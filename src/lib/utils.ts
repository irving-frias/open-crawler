import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
	return twMerge(clsx(inputs));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChild<T> = T extends { child?: any } ? Omit<T, "child"> : T;
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type WithoutChildren<T> = T extends { children?: any } ? Omit<T, "children"> : T;
export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & { ref?: U | null };

export async function notify(title: string, body: string): Promise<void> {
	try {
		const { isPermissionGranted, requestPermission, sendNotification } = await import("@tauri-apps/plugin-notification");
		let granted = await isPermissionGranted();
		if (!granted) {
			const permission = await requestPermission();
			granted = permission === "granted";
		}
		if (granted) {
			sendNotification({ title, body });
		}
	} catch {
		// Notifications not available
	}
}
