# GBC検証手順

2026-11-14時点。PRは機能ごとに積み上げており、前のPRをベースにしています。
Boot ROMや商用ゲームのバイナリは含めません。

## 自動テスト

```sh
cargo test
cargo build --example headless
```

通常実行と検証実行は `emulation::step_base_cycle` を共有します。
1呼び出しはPPUの4 dotsに相当し、倍速中はCPU/Timer/OAM DMAを2 M-cycle進めます。
検証実行では音声を破棄し、ウィンドウ・音声デバイス・実時間待機を使用しません。
1 ROMにつき1プロセスで実行してください。

## テストROMの実行と画面比較

```sh
cargo run --release --example headless -- path/to/test.gbc \
  --cgb-bootrom path/to/cgb_boot.bin \
  --cycles 10000000 --frames 300 --output actual.ppm

cargo run --release --example headless -- path/to/test.gbc \
  --cgb-bootrom path/to/cgb_boot.bin \
  --cycles 10000000 --frames 300 --expect expected.ppm
```

`--cycles` は通常速度のM-cycle単位の上限です。LCD停止や無限ループでも上限で終了します。
`--frames` は任意で、指定するとフレーム数に達した時点で停止し、上限までに達しなければ失敗します。
画像は160×144のRGB PPMです。比較はPPMファイル全体の完全一致で、不一致は終了コード1になります。
画面比較を指定しない正常終了は、ROMのテスト合格を意味しません。
ヘッドレス実行では通常のゲームセーブを読み込まず、保存もしません。

各結果には次を記録してください。

- エミュレータのコミットSHAと実行コマンド
- テストROMとBoot ROMの出典、バージョンまたはSHA256
- 実行上限、フレーム数、期待画像の出典とSHA256
- 終了コード、ログ、実際の画像、期待結果との差

## 現在の検証結果

| 対象 | 結果 |
| --- | --- |
| Boot ROM配置・ヘッダ領域・解除 | 合成データで成功 |
| CPU経由の合成CGB Boot ROM→カートリッジ引き渡し | 成功 |
| STOP中の割り込みと切り替え後の再開 | 合成ROMで成功 |
| GDMA中のCPU停止とTimer進行 | 倍速のスケジューラテストで成功 |
| HBlankごとの残量更新、VBlank中の停止 | 倍速のスケジューラテストで成功 |
| VRAM/WRAMバンクとEcho RAM | 成功 |
| BG属性・RGB出力・OBJ優先順位 | 単体テストで成功 |
| MBC5バンクとバッテリーセーブ再読み込み | 成功 |
| Headless CLIの画像出力・一致・不一致・上限 | 合成ROMで成功 |
| ユーザー所有のCGB Boot ROMによる起動 | ファイルパス未確認、未実施 |
| cgb-acid2の期待画像比較 | 未実施 |
| dmg-acid2と既存DMGゲームの回帰確認 | 未実施 |
| 実ゲームの音声・倍速・保存 | 未実施 |

## 既知の制限と次の確認

- 実CGB Boot ROMで起動を確認し、必要なレジスタや描画の不足を切り分ける。
- PPUは固定長の走査線モデル。描画中のレジスタ変更や厳密なアクセス競合は未再現。
- 速度切り替え中の特殊なPPUアクセス状態、DIV-APUのエッジは未再現。
- 未準備STOPの省電力停止は未対応。
- LCD無効中のHBlank DMA、HBlank中の開始、HALT境界は実機テストROMで追加確認する。
- DMA転送先オーバーフロー後のレジスタ状態は未解明なため、現実装では完了扱い。
- シリアル・赤外線・追加レジスタは対象ROMの必要性から別途対応する。

## 参考

- [Pan Docs](https://gbdev.io/pandocs/)
- [cgb-acid2](https://github.com/mattcurrie/cgb-acid2)
- [dmg-acid2](https://github.com/mattcurrie/dmg-acid2)
- [MagenTests](https://github.com/alloncm/MagenTests)
