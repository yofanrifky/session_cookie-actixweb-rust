# Restoran API

REST API restoran menggunakan Rust, Actix Web, dan PostgreSQL. Backend ini tidak menyertakan frontend. Login menggunakan session cookie; endpoint menu dan pesanan hanya dapat dipakai setelah login. Request dapat diuji dari Postman.

## Struktur Proyek

```text
backend/
|-- postman/Restoran-API.postman_collection.json
|-- src/
|   |-- handlers/
|   |   |-- auth.rs        # Register, login, logout, dan profil
|   |   |-- menus.rs       # Operasi menu
|   |   |-- orders.rs      # Operasi pesanan
|   |   |-- format.rs      # Helper format harga
|   |   `-- mod.rs         # Daftar modul handler
|   |-- auth_middleware.rs # Pemeriksaan session
|   |-- db.rs              # Persiapan tabel database
|   |-- main.rs            # Titik mulai aplikasi
|   |-- models.rs          # Bentuk data menu dan pesanan
|   |-- routes.rs          # Pendaftaran endpoint
|   `-- state.rs           # Pool database untuk handler
|-- catatan/               # Panduan belajar per file
|-- .env                   # Konfigurasi lokal, jangan commit
`-- Cargo.toml             # Dependency Rust
```

## Persiapan dan Menjalankan

1. Pastikan Rust/Cargo dan PostgreSQL sudah terpasang. Buat database kosong yang akan dipakai aplikasi.
2. Buat file `.env` pada folder backend. Isi `DATABASE_URL`, misalnya `postgres://USER:PASSWORD@HOST:PORT/NAMA_DATABASE`.
3. Tambahkan `SESSION_KEY` berisi 64 karakter acak. Di Linux/macOS, buat nilainya menggunakan `openssl rand -hex 32`. Jangan commit `.env` atau membagikan nilai rahasianya.
4. Jalankan aplikasi dari folder backend:

```sh
cargo run
```

Server default berjalan di `http://127.0.0.1:8080`. Saat startup, aplikasi membuat tabel `menus`, `orders`, dan `"user"` jika tabel belum tersedia. Tabel `"user"` memiliki kolom `id`, `username`, dan `password`.

`cargo check` hanya memeriksa kompilasi; perintah itu tidak menguji koneksi PostgreSQL. `cargo run` memerlukan database aktif dan nilai `.env` yang benar.

## Alur Login

1. Daftarkan akun melalui `POST /api/auth/register` dengan JSON `username` dan `password`.
2. Login melalui `POST /api/auth/login` menggunakan kredensial tadi.
3. Simpan cookie yang dikirim dalam response login. Postman perlu mengirim cookie yang sama pada request berikutnya.
4. Akses endpoint menu, pesanan, atau `GET /api/auth/me`.
5. Logout melalui `POST /api/auth/logout` untuk menghapus session.

Untuk latihan awal, password saat ini disimpan sebagai teks biasa di database agar alur register/login mudah dipahami. Ini tidak aman untuk aplikasi nyata: jangan gunakan password sungguhan, dan jangan deploy versi ini ke production. Endpoint register juga terbuka untuk siapa pun yang dapat mengakses server.

## Endpoint

| Metode | URL | Kegunaan | Login |
|---|---|---|---|
| POST | `/api/auth/register` | Membuat akun | Tidak |
| POST | `/api/auth/login` | Memeriksa password dan membuat session | Tidak |
| POST | `/api/auth/logout` | Menghapus session | Tidak wajib |
| GET | `/api/auth/me` | Melihat akun aktif | Ya |
| GET | `/api/menus` | Daftar menu atau satu kolom dengan `field` | Ya |
| GET | `/api/menus/{id}` | Detail satu menu | Ya |
| POST | `/api/menus` | Membuat menu | Ya |
| PUT | `/api/menus/{id}` | Mengganti seluruh field menu | Ya |
| PATCH | `/api/menus/{id}` | Mengganti sebagian field menu | Ya |
| DELETE | `/api/menus/{id}` | Menghapus menu | Ya |
| GET | `/api/orders` | Daftar pesanan atau satu kolom dengan `field` | Ya |
| GET | `/api/orders/{id}` | Detail satu pesanan | Ya |
| POST | `/api/orders` | Membuat pesanan | Ya |
| PUT | `/api/orders/{id}` | Mengganti seluruh field pesanan | Ya |
| PATCH | `/api/orders/{id}` | Mengganti sebagian field pesanan | Ya |
| DELETE | `/api/orders/{id}` | Menghapus pesanan | Ya |

Request ke endpoint yang perlu login akan ditolak dengan `401 Unauthorized` jika cookie session tidak dikirim atau tidak valid.

## Contoh Data

Body untuk membuat menu:

```json
{
  "name": "Nasi Goreng",
  "price": 25000,
  "category": "Makanan"
}
```

Harga disimpan dalam tipe PostgreSQL `NUMERIC(12,2)`, bukan floating-point. Angka `25000` akan ditampilkan sebagai `Rp25.000` oleh endpoint daftar/detail tertentu.

Body untuk membuat pesanan:

```json
{
  "customer_name": "Budi",
  "item_name": "Nasi Goreng",
  "quantity": 2,
  "status": "pending"
}
```

`PUT` memerlukan semua field input. `PATCH` hanya memerlukan field yang ingin diubah, contohnya:

```json
{
  "status": "diproses"
}
```

Pada `GET /api/menus`, parameter `field` yang didukung adalah `id`, `name`, `price`, dan `category`. Pada `GET /api/orders`, field-nya `id`, `customer_name`, `item_name`, `quantity`, dan `status`. Contoh: `GET /api/orders?field=status`.

## Postman

Impor `postman/Restoran-API.postman_collection.json`, jalankan server, lalu register dan login. Pastikan Postman menyimpan cookie login sebelum menjalankan request menu/pesanan. Sesuaikan ID pada request detail, update, dan delete dengan ID yang benar-benar ada.

Panduan menguji API dari terminal menggunakan `curl` ada di [catatan/akses_curl.md](catatan/akses_curl.md). Panduan pemula yang membahas implementasi per file ada di [catatan/README.md](catatan/README.md).