package com.example.zapas.core.navigation

import kotlinx.serialization.Serializable

@Serializable
object Splash

@Serializable
object Login

@Serializable
object Register

@Serializable
object Rooms

@Serializable
object CreateRoom

@Serializable
data class RoomProfile(val roomId: String)

@Serializable
object Profile

@Serializable
data class MemberProfile(val memberId: String)
