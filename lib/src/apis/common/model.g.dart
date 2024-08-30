// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'model.dart';

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

PageList<T> _$PageListFromJson<T>(
  Map<String, dynamic> json,
  T Function(Object? json) fromJsonT,
) =>
    PageList<T>(
      (json['data'] as List<dynamic>).map(fromJsonT).toList(),
      json['has_next'] as bool,
      (json['page'] as num).toInt(),
      (json['page_size'] as num).toInt(),
    );

Map<String, dynamic> _$PageListToJson<T>(
  PageList<T> instance,
  Object? Function(T value) toJsonT,
) =>
    <String, dynamic>{
      'data': instance.data.map(toJsonT).toList(),
      'has_next': instance.hasNext,
      'page': instance.page,
      'page_size': instance.pageSize,
    };
