# poc_pcd

pcd 形式のファイルを出力の検討

## src/bin

### make_dummy

ダミーデータの作成

[バイナリ形式の pcd 形式のファイルを出力する - Qiita](https://qiita.com/gou_koutaki/items/3c430db5e99e8771ed94)を参考にした

### convert

LJX から取得した生データの変換

以下の課題がある

- ヘッダの描画点数をデータ変更後に書き換える必要がある
- 描画点数が多い ⇒ プロファイルの真ん中だけ抜き取りたい

## extract_convert

抽出範囲指定を追加。輝度データを含むプロファイルに対応

pcd 形式は上手くいかないので csv で検討 ⇒ 輝度データ部が出力に紛れ込んでたかも
