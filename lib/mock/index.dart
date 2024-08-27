import 'package:dio/dio.dart';
import 'package:http_mock_adapter/http_mock_adapter.dart';

void startMockDio(Dio dio) {
  final dioAdapter = DioAdapter(dio: dio);

  dioAdapter.onGet(
    "/api/vi/user/login",
    (server) => server.reply(
      200,
      {'message': 'success', "code": 200, "result": "success"},
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
          "create_at": "",
          "update_at": ""
        }
      },
      delay: const Duration(seconds: 1),
    ),
  );
}
