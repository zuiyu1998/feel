// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'model.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

UserBaseModel _$UserBaseModelFromJson(Map<String, dynamic> json) =>
    UserBaseModel(
      (json['id'] as num).toInt(),
      json['nikename'] as String,
      json['uid'] as String,
      json['avatar'] as String,
      DateTime.parse(json['create_at'] as String),
      DateTime.parse(json['update_at'] as String),
    );

Map<String, dynamic> _$UserBaseModelToJson(UserBaseModel instance) =>
    <String, dynamic>{
      'id': instance.id,
      'nikename': instance.nikename,
      'uid': instance.uid,
      'avatar': instance.avatar,
      'create_at': instance.createAt.toIso8601String(),
      'update_at': instance.updateAt.toIso8601String(),
    };
