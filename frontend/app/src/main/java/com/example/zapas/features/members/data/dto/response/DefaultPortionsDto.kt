package com.example.zapas.features.members.data.dto.response

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class DefaultPortionsDto(
    @Json(name = "portions") val portions: Long
)
