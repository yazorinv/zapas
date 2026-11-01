package com.example.zapas.features.members.domain.repository

import arrow.core.Either
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.members.domain.model.MemberPortion
import com.example.zapas.features.members.domain.model.LocalMemberProfile
import com.example.zapas.features.rooms.domain.model.Member

interface MembersRepository {
    /** Display profile: local values fall back to the global user profile. */
    suspend fun getMember(memberId: String): Either<AppError, Member>
    suspend fun getLocalProfile(memberId: String): Either<AppError, LocalMemberProfile>
    suspend fun updateProfile(
        memberId: String, displayName: String?, bio: String?
    ): Either<AppError, LocalMemberProfile>
    suspend fun getDefaultPortions(memberId: String): Either<AppError, Long>
    suspend fun setDefaultPortions(memberId: String, portions: Long): Either<AppError, Long>
    /** Dates are sent unchanged; range boundary semantics belong to the server. */
    suspend fun getPortions(
        memberId: String, begin: String, end: String
    ): Either<AppError, List<MemberPortion>>
    suspend fun setPortion(memberId: String, date: String, portions: Long): Either<AppError, Unit>
}
