# EcoAlerta 🌱 - Frontend Rust + WebAssembly (Wasm)

Este módulo es una implementación de alto rendimiento para el frontend de **EcoAlerta** en **Cerro de Pasco, Perú**, compilado a **WebAssembly nativo** utilizando **Rust**, **Leptos** y **Trunk**.

---

## ⚡ Beneficios frente a Flutter Web
- **Peso de descarga mínimo:** Pasa de ~8-12 MB (CanvasKit de Flutter) a menos de **800 KB** (Wasm + HTML/CSS).
- **Arranque instantáneo:** Carga en menos de 1 segundo incluso en conexiones móviles 3G/4G de Pasco.
- **Renderizado nativo:** Utiliza el DOM HTML y la GPU del navegador con Tailwind CSS (Glassmorphism fluido).
- **Mapa Leaflet optimizado:** Control reactivo de OpenStreetMap con capas claras y oscuras (CartoDB Dark).

---

## 🛠️ Requisitos
- **Rust y Cargo** (1.75+)
- Target WebAssembly:
  ```bash
  rustup target add wasm32-unknown-unknown
  ```
- **Trunk** (Empaquetador Wasm):
  ```bash
  curl -sL https://github.com/trunk-rs/trunk/releases/download/v0.22.0/trunk-x86_64-apple-darwin.tar.gz | tar -xz -C ~/.cargo/bin/
  ```

---

## 🚀 Ejecución en Desarrollo
Para iniciar el servidor local con recarga en vivo (*hot-reload*):
```bash
cd frontend-wasm
trunk serve
```
Abre en tu navegador: `http://127.0.0.1:8080`

---

## 📦 Compilación para Producción
```bash
cd frontend-wasm
trunk build --release
```
Los archivos optimizados se generarán en la carpeta `dist/` listos para servirse con Nginx, Coolify o cualquier servidor estático.
