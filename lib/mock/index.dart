import 'package:dio/dio.dart';
import 'package:http_mock_adapter/http_mock_adapter.dart';

void startMockDio(Dio dio) {
  final dioAdapter = DioAdapter(dio: dio);

  dioAdapter.onGet(
    "/api/vi/user/login",
    (server) => server.reply(
      200,
      {
        'message': 'success',
        "code": 200,
        "result": {"token": "1111"}
      },
      delay: const Duration(seconds: 1),
    ),
  );

  dioAdapter.onGet(
    "/api/vi/user/get_base_user_info",
    (server) => server.reply(
      200,
      {
        'message': 'success',
        "code": 200,
        "result": {
          "id": 1,
          "nikename": "test",
          "avatar": "11111",
          "uid": "123456",
          "create_at": "2019-01-25T02:00:00.000Z",
          "update_at": "2019-01-25T02:00:00.000Z"
        }
      },
      delay: const Duration(seconds: 1),
    ),
  );
}
