## GameBoy Emulator written in Rust.  
  
Bootrom → [here.](https://github.com/take44444/Gameboy-free_bootrom.git)
CGB Boot ROMを使う場合:

```sh
cargo run --release -- path/to/game.gbc --cgb-bootrom path/to/cgb_boot.bin
```

Boot ROM未指定時は従来の組み込みDMG Boot ROMを使用します。CGB Boot ROMは
0x900バイト（ヘッダ領域の空きを含む形式）または0x800バイト（空きを除く形式）を
指定できます。ROMファイルはリポジトリに含めません。

起動・マッピングのテスト:

```sh
SDL_AUDIODRIVER=dummy cargo test
```

CGBの描画・DMA・倍速対応は開発中です。外部Boot ROMの読み込みに対応したことは、
すべてのCGBゲームの起動・表示を保証するものではありません。
