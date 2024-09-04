// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'model.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

FeedSource _$FeedSourceFromJson(Map<String, dynamic> json) => FeedSource(
      (json['id'] as num).toInt(),
      $enumDecode(_$FeedKindEnumMap, json['kind']),
      UserBaseModel.fromJson(json['user'] as Map<String, dynamic>),
    );

Map<String, dynamic> _$FeedSourceToJson(FeedSource instance) =>
    <String, dynamic>{
      'id': instance.id,
      'kind': _$FeedKindEnumMap[instance.kind]!,
      'user': instance.user,
    };

const _$FeedKindEnumMap = {
  FeedKind.post: 'post',
};

Feed _$FeedFromJson(Map<String, dynamic> json) => Feed(
      (json['id'] as num).toInt(),
      FeedSource.fromJson(json['source'] as Map<String, dynamic>),
      json['summary'] as Map<String, dynamic>,
      DateTime.parse(json['create_at'] as String),
      DateTime.parse(json['update_at'] as String),
      (json['like'] as num).toInt(),
      (json['comment'] as num).toInt(),
      (json['share'] as num).toInt(),
    );

Map<String, dynamic> _$FeedToJson(Feed instance) => <String, dynamic>{
      'id': instance.id,
      'source': instance.source,
      'summary': instance.summary,
      'create_at': instance.createAt.toIso8601String(),
      'update_at': instance.updateAt.toIso8601String(),
      'like': instance.like,
      'share': instance.share,
      'comment': instance.comment,
    };
