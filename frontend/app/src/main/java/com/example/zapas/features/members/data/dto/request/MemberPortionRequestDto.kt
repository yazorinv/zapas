package com.example.zapas.features.members.data.dto.request

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class MemberPortionRequestDto(
    @Json(name = "date") val date: String,
    @Json(name = "portions") val portions: Long
)
