package com.example.zapas.features.rooms.domain.model

data class Room(
    val bio: String,
    val createdAt: String,
    val displayName: String,
    val id: String,
    val expiredNotify: Long?,
    val reviewInterval: Long?
)