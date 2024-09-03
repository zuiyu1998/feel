// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'model.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

Post _$PostFromJson(Map<String, dynamic> json) => Post(
      (json['id'] as num).toInt(),
      json['content'] as String,
      (json['images'] as List<dynamic>).map((e) => e as String).toList(),
      DateTime.parse(json['create_at'] as String),
      DateTime.parse(json['update_at'] as String),
      (json['useId'] as num).toInt(),
    );

Map<String, dynamic> _$PostToJson(Post instance) => <String, dynamic>{
      'id': instance.id,
      'content': instance.content,
      'images': instance.images,
      'create_at': instance.createAt.toIso8601String(),
      'update_at': instance.updateAt.toIso8601String(),
      'useId': instance.useId,
    };

PostSummary _$PostSummaryFromJson(Map<String, dynamic> json) => PostSummary(
      json['content'] as String?,
      (json['images'] as List<dynamic>?)?.map((e) => e as String).toList(),
      DateTime.parse(json['create_at'] as String),
      DateTime.parse(json['update_at'] as String),
    );

Map<String, dynamic> _$PostSummaryToJson(PostSummary instance) =>
    <String, dynamic>{
      'content': instance.content,
      'images': instance.images,
      'create_at': instance.createAt.toIso8601String(),
      'update_at': instance.updateAt.toIso8601String(),
    };
