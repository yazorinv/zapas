package com.example.zapas.features.members.data.dto.response

import com.example.zapas.core.network.dto.ResponseDto
import com.example.zapas.features.members.domain.model.LocalMemberProfile
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class OriginMemberDto(
    @Json(name = "id") val id: String,
    @Json(name = "room_id") val roomId: String,
    @Json(name = "created_at") val createdAt: String,
    @Json(name = "user_id") val userId: String? = null,
    @Json(name = "display_name") val displayName: String? = null,
    @Json(name = "bio") val bio: String? = null
) : ResponseDto<LocalMemberProfile> {
    override fun toDomain() = LocalMemberProfile(id, roomId, userId, createdAt, displayName, bio)
}
