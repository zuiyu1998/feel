// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'user.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

UserStore _$UserStoreFromJson(Map<String, dynamic> json) => UserStore()
  ..token = json['token'] as String?
  ..user = json['user'] == null
      ? null
      : UserBaseModel.fromJson(json['user'] as Map<String, dynamic>);

Map<String, dynamic> _$UserStoreToJson(UserStore instance) => <String, dynamic>{
      'token': instance.token,
      'user': instance.user,
    };
