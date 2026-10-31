package com.example.zapas.features.rooms.data.dto.response

import com.example.zapas.core.network.dto.ResponseDto
import com.example.zapas.features.rooms.domain.model.Room
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
class RoomProfileDto(
    @Json(name = "bio") val bio: String,
    @Json(name = "created_at") val createdAt: String,
    @Json(name = "display_name") val displayName: String,
    @Json(name = "id") val id: String,
    @Json(name = "expired_notify") val expiredNotify: Long,
    @Json(name = "review_interval") val reviewInterval: Long
) : ResponseDto<Room> {
    override fun toDomain() = Room(
        bio = bio,
        createdAt = createdAt,
        displayName = displayName,
        id = id,
        expiredNotify = expiredNotify,
        reviewInterval = reviewInterval
    )
}