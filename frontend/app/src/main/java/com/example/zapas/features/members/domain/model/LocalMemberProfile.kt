package com.example.zapas.features.members.domain.model

data class LocalMemberProfile(
    val id: String,
    val roomId: String,
    val userId: String?,
    val createdAt: String,
    val displayName: String?,
    val bio: String?
)
