package com.example.zapas.features.rooms.data.dto.request

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
class UpdateRoomTimingsDto(
    @Json(name = "expired_notify") val expiredNotify: Long?,
    @Json(name = "review_interval") val reviewInterval: Long?
)