package com.example.zapas.features.members.data.repository

import com.example.zapas.core.network.NetworkErrorMapper
import com.example.zapas.features.members.data.api.MembersApiService
import com.example.zapas.features.members.data.dto.request.DefaultPortionsRequestDto
import com.example.zapas.features.members.data.dto.request.MemberPortionRequestDto
import com.example.zapas.features.members.data.dto.request.UpdateMemberProfileDto
import com.example.zapas.features.members.domain.repository.MembersRepository
import javax.inject.Inject

class MembersRepositoryImpl @Inject constructor(
    private val api: MembersApiService,
    private val errorMapper: NetworkErrorMapper
) : MembersRepository {
    override suspend fun getMember(memberId: String) =
        errorMapper.safeApiCallMapped { api.member(memberId) }

    override suspend fun getLocalProfile(memberId: String) =
        errorMapper.safeApiCallMapped { api.origin(memberId) }

    override suspend fun updateProfile(memberId: String, displayName: String?, bio: String?) =
        errorMapper.safeApiCallMapped {
            api.updateProfile(memberId, UpdateMemberProfileDto(displayName, bio))
        }

    override suspend fun getDefaultPortions(memberId: String) =
        errorMapper.safeApiCall { api.defaultPortions(memberId) }.map { it.portions }

    override suspend fun setDefaultPortions(memberId: String, portions: Long) =
        errorMapper.safeApiCall { api.setDefaultPortions(memberId, DefaultPortionsRequestDto(portions)) }
            .map { it.portions }

    override suspend fun getPortions(memberId: String, begin: String, end: String) =
        errorMapper.safeApiCall { api.portions(memberId, begin, end) }
            .map { records -> records.map { it.toDomain() } }

    override suspend fun setPortion(memberId: String, date: String, portions: Long) =
        errorMapper.safeApiCall { api.setPortion(memberId, MemberPortionRequestDto(date, portions)) }
}
