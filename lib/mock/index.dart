import 'package:dio/dio.dart';
import 'package:feel/mock/feed.dart';
import 'package:feel/mock/post.dart';
import 'package:feel/mock/user.dart';
import 'package:http_mock_adapter/http_mock_adapter.dart';

void startMockDio(Dio dio) {
  final dioAdapter = DioAdapter(dio: dio);

  buildFeedApiMock(dioAdapter);
  buildUserApiMock(dioAdapter);
  buildPostApiMock(dioAdapter);
}
