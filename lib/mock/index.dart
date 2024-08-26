import 'package:dio/dio.dart';
import 'package:flutter/material.dart';
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
        "result": {"token": "success"}
      },
      delay: const Duration(seconds: 1),
    ),
  );

  debugPrint("startMockDio");
}
