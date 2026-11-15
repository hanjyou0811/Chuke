## GameBoy Emulator written in Rust.  
  
Bootrom → [here.](https://github.com/take44444/Gameboy-free_bootrom.git)
通常は組み込みCGB Boot ROMを使用します。別のCGB Boot ROMを指定する場合:

```sh
cargo run --release -- path/to/game.gbc --cgb-bootrom path/to/cgb_boot.bin
```

Boot ROM未指定時は2304バイトのCGB Boot ROMを使用します。外部CGB Boot ROMは
0x900バイト（ヘッダ領域の空きを含む形式）または0x800バイト（空きを除く形式）を
指定できます。組み込みBoot ROMは `src/bootrom.rs` のバイト配列に直接記載しています。

起動・マッピングのテスト:

```sh
SDL_AUDIODRIVER=dummy cargo test
```

CGBの描画・DMA・倍速対応は開発中です。外部Boot ROMの読み込みに対応したことは、
すべてのCGBゲームの起動・表示を保証するものではありません。
