package com.example.zapas.features.members.data.dto

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

/** Nullable room profile fields from the API; empty strings are preserved. */
@JsonClass(generateAdapter = true)
data class UpdateMemberProfileDto(
    @Json(name = "display_name") val displayName: String? = null,
    val bio: String? = null
)
