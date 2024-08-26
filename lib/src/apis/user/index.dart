import 'package:feel/src/utils/http/index.dart';

Future<dynamic> login() async {
  var res = await defineDio.get("/api/vi/user/login");

  return res["token"];
}
