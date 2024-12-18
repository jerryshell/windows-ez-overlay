# Windows EZ Overlay for Rust 🦀

Windows GDI overlay written in Rust.

The goal is to keep it simple and easy to use.

## How to use

1. Cargo.toml

```toml
windows-ez-overlay = { git = "https://github.com/jerryshell/windows-ez-overlay" }
```

2. Rust code

```rust
// init draw_rect_list, ez-overlay will draw Rectangle using draw_rect_list
let draw_rect_list = Arc::new(RwLock::new(Vec::<RECT>::with_capacity(32)));

// get target window info
let game_window = FindWindowA(None, s!("AssaultCube"));
let mut window_info = WINDOWINFO::default();
GetWindowInfo(game_window, &mut window_info);

// init ez-overlay with window_info and draw_rect_list
const FRAME_RATE: u64 = 60;
let draw_rect_list_clone = Arc::clone(&draw_rect_list);
std::thread::spawn(move || {
    let mut overlay = windows_ez_overlay::Overlay::new(
        window_info.rcClient.left,
        window_info.rcClient.top,
        window_info.rcClient.right,
        window_info.rcClient.bottom,
        draw_rect_list_clone,
        FRAME_RATE,
        true,
    );
    let _ = overlay.window_loop();
});

// ... do your stuff ...

{
    let mut draw_rect_list = draw_rect_list.write().unwrap();
    // update draw_rect_list from here
}
```

## Test

```bash
cargo test
```

## Example

[ac-esp | AssaultCube ESP DLL with Rust 🦀](https://github.com/jerryshell/ac-esp)

## License

[GNU Affero General Public License v3.0](LICENSE)
