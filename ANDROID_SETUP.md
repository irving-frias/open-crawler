# Android Build Setup

## Keystore de Producción

El keystore de producción se generó en: `android/opencrawler.keystore`

**IMPORTANTE**: Guardá este archivo en un lugar seguro (por ejemplo un gestor de
passwords o un almacén cifrado). Es la única copia — si se pierde, no podrás
publicar actualizaciones del APK firmado. El keystore de dev antiguo quedó
respaldado como `android/opencrawler.keystore.dev.bak`.

El password del keystore de producción NO está en este archivo por seguridad.
Se generó con `tools/create-android-keystore.sh` y se configuró directamente en
los secrets de GitHub.

Para (re)generar el keystore y obtener los valores de los secrets:

```bash
tools/create-android-keystore.sh --alias <alias> --password <password>
```

O manualmente (requiere Java, o Docker si no tenés `keytool`):

```bash
keytool -genkeypair -v -keystore android/opencrawler.keystore -alias <alias> -keyalg RSA -keysize 2048 -validity 10000 -storepass <password> -keypass <password> -dname "CN=Open Crawler, OU=Development, O=Open Crawler, L=City, ST=State, C=US"
```

El keystore se genera en formato PKCS12 (soportado por AGP); el nombre del archivo es solo una convención.

## Secrets de GitHub

Andá a tu repositorio → Settings → Secrets and variables → Actions. Los valores
de producción ya están configurados (alias `opencrawler-prod`):

| Secret                      | Value               |
| --------------------------- | ------------------- |
| `ANDROID_KEYSTORE_BASE64`   | Base64 del keystore |
| `ANDROID_KEY_ALIAS`         | `opencrawler-prod`  |
| `ANDROID_KEYSTORE_PASSWORD` | (generado)          |
| `ANDROID_KEY_PASSWORD`      | (generado)          |

Para obtener el base64 del keystore:

```bash
base64 -i android/opencrawler.keystore | pbcopy  # macOS
# o
base64 -w 0 android/opencrawler.keystore        # Linux
```

## Workflow

El build de Android está en `.github/workflows/release.yml` (job `android`).

Se ejecuta automáticamente en:

- Tag `v*` (publica APK/AAB firmados en el release)
- Manualmente con `workflow_dispatch`

## Build Local

Para buildear localmente:

```bash
bun run tauri build --target aarch64-linux-android
```

El APK se genera en: `src-tauri/target/android/arm64-v8a/release/apk/`

## Notas

- El APK se sube como artifact de GitHub Actions por 30 días
- El keystore actual es de **producción** (alias `opencrawler-prod`); el de dev
  quedó en `android/opencrawler.keystore.dev.bak`
- El build local de Android requiere OpenSSL y Android NDK configurados
