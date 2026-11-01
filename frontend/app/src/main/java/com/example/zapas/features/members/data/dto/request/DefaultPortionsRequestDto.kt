package com.example.zapas.features.members.data.dto.request

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class DefaultPortionsRequestDto(
    @Json(name = "portions") val portions: Long
)
