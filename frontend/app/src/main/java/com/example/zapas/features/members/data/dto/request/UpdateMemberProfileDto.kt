package com.example.zapas.features.members.data.dto.request

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass
@JsonClass(generateAdapter = true)
data class UpdateMemberProfileDto(
    @Json(name = "display_name") val displayName: String? = null,
    @Json(name = "bio") val bio: String? = null
)
