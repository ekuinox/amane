'''
Amaneサーバにリクエストするサンプル
コマンドライン引数からホスト名, バケット名, キー, 対象ファイルのパスの順に受け取る
'''
from sys import argv
from .amane import Amane

if __name__ != '__main__':
    exit(0)

if len(argv) < 5:
    exit(-1)

host, bucket, key, path = argv[1:]

amane = Amane(host)

with open(path) as file:
    amane.put(bucket, key, file, meta=[('agent', 'request_image_file')])
