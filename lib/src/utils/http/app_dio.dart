import 'package:dio/dio.dart';
import 'package:feel/src/store/modules/user.dart';
import 'package:get/get.dart' show Get, Inst;

abstract class AppTransformer {
  Map<String, dynamic> onResponse(Response res);
}

class DefaultAppTransformer implements AppTransformer {
  @override
  Map<String, dynamic> onResponse(Response res) {
    if (res.data["code"] == 200) {
      return res.data["result"];
    } else {
      throw DioException.badResponse(
          statusCode: res.data["code"],
          requestOptions: res.requestOptions,
          response: res);
    }
  }
}

class AppTransformerInterceptor extends Interceptor {
  final AppTransformer transformer;

  AppTransformerInterceptor(
    this.transformer,
  );

  @override
  void onRequest(
    RequestOptions options,
    RequestInterceptorHandler handler,
  ) {
    var userStore = Get.find<UserStore>();
    var token = userStore.getToken();

    if (token != null) {
      options.headers["Authorization"] = "Beare $token";
    }

    handler.next(options);
  }

  @override
  void onResponse(Response response, ResponseInterceptorHandler handler) {
    try {
      response.data = transformer.onResponse(response);
      handler.next(response);
    } on DioException catch (e) {
      handler.reject(e);
    }
  }
}

class AppDio {
  late Dio dio;

  AppDio({AppTransformerInterceptor? transformer, BaseOptions? options}) {
    dio = Dio(options);

    dio.interceptors
        .add(transformer ?? AppTransformerInterceptor(DefaultAppTransformer()));
  }

  Future<T?> get<T>(
    String url, {
    Map<String, dynamic>? queryParameters,
  }) async {
    final Response<T> res =
        await requestInteral(url, queryParameters: queryParameters);
    return res.data;
  }

  //错误处理
  void onError(DioException e) {}

  Future<Response<T>> requestInteral<T>(
    String url, {
    Object? data,
    Map<String, dynamic>? queryParameters,
    CancelToken? cancelToken,
    Options? options,
    ProgressCallback? onSendProgress,
    ProgressCallback? onReceiveProgress,
  }) async {
    try {
      return await dio.request(url,
          data: data,
          queryParameters: queryParameters,
          cancelToken: cancelToken,
          options: options,
          onSendProgress: onSendProgress,
          onReceiveProgress: onReceiveProgress);
    } on DioException catch (e) {
      onError(e);
      rethrow;
    }
  }
}
