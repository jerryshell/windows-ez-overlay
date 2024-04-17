# Windows EZ Overlay for Rust 🦀

Windows GDI overlay written in Rust.

The goal is to keep it simple and easy to use.

## How to use

1. Cargo.toml

```toml
windows-ez-overlay = { git = "https://github.com/jerryshell/windows-ez-overlay.git" }
```

2. Rust code

```rust
// init draw_rect_list, ez-overlay will draw Rectangle using draw_rect_list
let draw_rect_list = Arc::new(RwLock::new(Vec::<RECT>::with_capacity(32)));

// get target window info
let game_window = FindWindowA(None, s!("AssaultCube"));
let mut window_info = WINDOWINFO::default();
GetWindowInfo(game_window, &mut window_info)?;

// init ez-overlay with window_info and draw_rect_list
let refresh_interval_ms = 1000 / 60;
let draw_rect_list_clone = Arc::clone(&draw_rect_list);
std::thread::spawn(move || {
    let mut overlay = windows_ez_overlay::Overlay::new(
        window_info.rcClient.left,
        window_info.rcClient.top,
        window_info.rcClient.right,
        window_info.rcClient.bottom,
        refresh_interval_ms,
        draw_rect_list_clone,
        true,
    );
    let _ = overlay.window_loop();
});

// update draw_rect_list from here
// ...
```

## Test

```bash
cargo t
```

## Example

[ac-esp | AssaultCube ESP DLL with Rust 🦀](https://github.com/jerryshell/ac-esp)

## LICENSE

[GNU Affero General Public License v3.0](https://choosealicense.com/licenses/agpl-3.0/)
