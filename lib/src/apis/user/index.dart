import 'package:feel/src/store/modules/user.dart';
import 'package:feel/src/utils/http/index.dart';

class UserApi {
  static Future<String> userLogin() async {
    var res = await defineDio.get("/api/vi/user/login");

    return res["token"];
  }

  static Future<UserBaseModel> getBaseUserInfo() async {
    var res = await defineDio.get("/api/vi/user/get_base_user_info");
    return UserBaseModel.fromJson(res);
  }
}
