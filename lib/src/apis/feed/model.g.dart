// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'model.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

FeedSource _$FeedSourceFromJson(Map<String, dynamic> json) => FeedSource(
      (json['id'] as num).toInt(),
      $enumDecode(_$FeedKindEnumMap, json['kind']),
    );

Map<String, dynamic> _$FeedSourceToJson(FeedSource instance) =>
    <String, dynamic>{
      'id': instance.id,
      'kind': _$FeedKindEnumMap[instance.kind]!,
    };

const _$FeedKindEnumMap = {
  FeedKind.post: 'post',
};

Feed _$FeedFromJson(Map<String, dynamic> json) => Feed(
      (json['id'] as num).toInt(),
      FeedSource.fromJson(json['source'] as Map<String, dynamic>),
      json['summary'] as Map<String, dynamic>,
    );

Map<String, dynamic> _$FeedToJson(Feed instance) => <String, dynamic>{
      'id': instance.id,
      'source': instance.source,
      'summary': instance.summary,
    };
