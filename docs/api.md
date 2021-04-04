# こういう風なAPIにしたい

## ファイルの作成 & 更新

`PUT /{bucketName}/{fileKey}`

multipart/form-data でファイルを投げる

## ファイルの取得

`GET /{bucketName}/{fileKey}`

## ファイルの削除

`DELETE /{bucketName}/{fileKey}`

## ファイルの一覧取得

`GET /{bucketName}/{prefix}`

prefixで始まるfileKeyの一覧を取得する

**できれば実装する...**
