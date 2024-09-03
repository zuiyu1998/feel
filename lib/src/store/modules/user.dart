import 'dart:convert';

import 'package:feel/src/apis/user/index.dart';
import 'package:feel/src/apis/user/model.dart';
import 'package:feel/src/helpers/shared_preferences_helper.dart';
import 'package:feel/src/store/consts.dart';
import 'package:feel/src/store/store.dart';
import 'package:json_annotation/json_annotation.dart';

part 'user.g.dart';

@JsonSerializable()
class UserStore extends Store {
  UserStore();

  //token
  String? token;

  //用户信息
  UserBaseModel? user;

  bool isAuth() {
    if (token != null) {
      return true;
    } else {
      return false;
    }
  }

  String? getToken() {
    return token;
  }

  Future<void> setToken(String? token) async {
    this.token = token;
    await save();
  }

  ///登录
  Future<void> login() async {
    var token = await UserApi.userLogin();
    setToken(token);

    await getBaseUserInfo();
  }

  ///获取用户的基础信息
  Future<void> getBaseUserInfo() async {
    var user = await UserApi.getBaseUserInfo();
    this.user = user;
    await save();
  }

  @override
  Future<void> save() async {
    await SharedPreferencesHelper.setString(userKey, json.encode(toJson()));
  }

  @override
  Future<void> ensureInitialized() async {
    var userStr = await SharedPreferencesHelper.getString(userKey);
    if (userStr != null) {
      var userJson = json.decode(userStr);
      var useStore = UserStore.fromJson(userJson);

      token = useStore.token;
      user = useStore.user;
    }
  }

  factory UserStore.fromJson(Map<String, dynamic> json) =>
      _$UserStoreFromJson(json);

  Map<String, dynamic> toJson() => _$UserStoreToJson(this);
}
