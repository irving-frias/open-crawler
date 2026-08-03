# Android Build Setup

## Keystore Generado

El keystore se generó en: `android/opencrawler.keystore`

**IMPORTANTE**: Este keystore es para desarrollo/testing. Para producción, generá uno nuevo con tus datos reales.

```bash
keytool -genkey -v -keystore android/opencrawler.keystore -alias opencrawler -keyalg RSA -keysize 2048 -validity 10000
```

## Secrets de GitHub

Andá a tu repositorio → Settings → Secrets and variables → Actions, y agregá estos secrets:

| Secret | Value |
|--------|-------|
| `ANDROID_KEYSTORE_BASE64` | Base64 del keystore |
| `ANDROID_KEY_ALIAS` | `opencrawler` |
| `ANDROID_KEYSTORE_PASSWORD` | `android` |
| `ANDROID_KEY_PASSWORD` | `android` |

Para obtener el base64 del keystore:

```bash
base64 -i android/opencrawler.keystore | pbcopy  # macOS
# o
base64 -w 0 android/opencrawler.keystore        # Linux
```

## Workflow

El workflow está en `.github/workflows/android.yml`.

Se ejecuta automáticamente en:
- Push a `main`
- Pull requests a `main`
- Manualmente con `workflow_dispatch`

## Build Local

Para buildear localmente:

```bash
bun run tauri build --target aarch64-linux-android
```

El APK se genera en: `src-tauri/target/android/arm64-v8a/release/apk/`

## Notas

- El APK se sube como artifact de GitHub Actions por 30 días
- El keystore actual tiene password `android` (cambialo para producción)
- El alias es `opencrawler`
- El build local de Android requiere OpenSSL y Android NDK configurados
