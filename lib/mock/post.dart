import 'package:http_mock_adapter/http_mock_adapter.dart';

void buildPostApiMock(DioAdapter dioAdapter) {
  dioAdapter.onGet(
    "/api/v1/post/get_post",
    (server) => server.reply(
      200,
      {
        'message': 'success',
        "code": 200,
        "result": {
          "id": 0,
          "content": "test",
          "images": [],
          "create_at": "2019-01-25T02:00:00.000Z",
          "update_at": "2019-01-25T02:00:00.000Z",
          "user": {
            "id": 1,
            "nikename": "test",
            "avatar":
                "https://i1.hdslb.com/bfs/face/14686527d8d715e986ddd29a208775a29a0030c8.jpg",
            "uid": "123456",
            "create_at": "2019-01-25T02:00:00.000Z",
            "update_at": "2019-01-25T02:00:00.000Z"
          }
        }
      },
      delay: const Duration(seconds: 1),
    ),
  );
}
