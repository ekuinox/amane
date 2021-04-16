import requests
from typing import IO, Any

class Amane:
    '''
    Amaneサーバにリクエストするクライアント
    '''
    _host: str # 
    
    def __init__(self, host: str):
        self._host = host

    def put(self, bucket: str, key: str, file: IO[Any], meta: list[(str, str)]):
        '''
        ファイルをアップロードする
        '''
        headers = dict([('x-amn-meta-{}'.format(k), v) for (k, v) in meta])
        return requests.put(
            'http://{}/{}/{}'.format(self._host, bucket, key),
            files={ 'file': file },
            headers=headers
        )
