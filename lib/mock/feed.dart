import 'package:http_mock_adapter/http_mock_adapter.dart';

void buildFeedApiMock(DioAdapter dioAdapter) {
  dioAdapter.onGet(
    "/api/v1/feed/get_feed_page_list",
    (server) => server.reply(
      200,
      {
        'message': 'success',
        "code": 200,
        "result": {
          "has_next": false,
          "page": 1,
          "page_size": 50,
          "data": [
            {
              "id": 1,
              "source": {
                "id": 1,
                "kind": "post",
                "user": {
                  "id": 1,
                  "nikename": "test",
                  "avatar":
                      "https://i1.hdslb.com/bfs/face/14686527d8d715e986ddd29a208775a29a0030c8.jpg",
                  "uid": "123456",
                  "create_at": "2019-01-25T02:00:00.000Z",
                  "update_at": "2019-01-25T02:00:00.000Z"
                }
              },
              "summary": {
                "content": "这是测试",
                "create_at": "2019-01-25T02:00:00.000Z",
                "update_at": "2019-01-25T02:00:00.000Z",
                "imags": []
              },
              "create_at": "2019-01-25T02:00:00.000Z",
              "update_at": "2019-01-25T02:00:00.000Z",
              "like": 1,
              "share": 0,
              "comment": 2
            },
            {
              "id": 2,
              "source": {
                "id": 1,
                "kind": "post",
                "user": {
                  "id": 1,
                  "nikename": "test",
                  "avatar":
                      "https://i1.hdslb.com/bfs/face/14686527d8d715e986ddd29a208775a29a0030c8.jpg",
                  "uid": "123456",
                  "create_at": "2019-01-25T02:00:00.000Z",
                  "update_at": "2019-01-25T02:00:00.000Z"
                }
              },
              "summary": {
                "content": "这是测试1",
                "create_at": "2019-01-25T02:00:00.000Z",
                "update_at": "2019-01-25T02:00:00.000Z",
                "images": []
              },
              "create_at": "2019-01-25T02:00:00.000Z",
              "update_at": "2019-01-25T02:00:00.000Z",
              "like": 1,
              "share": 0,
              "comment": 2
            }
          ]
        }
      },
      delay: const Duration(seconds: 1),
    ),
  );
}
