package com.example.zapas.features.rooms.data.repository

import androidx.paging.Pager
import androidx.paging.PagingConfig
import androidx.paging.PagingData
import arrow.core.Either
import com.example.zapas.core.network.NetworkErrorMapper
import com.example.zapas.features.rooms.data.repository.paging.RoomsPagingSource
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.core.utils.PagingPage
import com.example.zapas.features.rooms.domain.model.Room
import com.example.zapas.features.rooms.domain.repository.RoomsRepository
import com.example.zapas.features.rooms.data.api.RoomsApiService
import com.example.zapas.features.rooms.data.dto.request.CreateRoomDto
import com.example.zapas.features.rooms.data.dto.request.UpdateRoomDto
import com.example.zapas.features.rooms.data.dto.request.UpdateRoomTimingsDto
import com.example.zapas.features.rooms.data.repository.paging.MembersPagingSource
import com.example.zapas.features.rooms.domain.model.Member
import kotlinx.coroutines.flow.Flow
import javax.inject.Inject

class RoomsRepositoryImpl @Inject constructor(
    private val roomApi: RoomsApiService, private val networkErrorMapper: NetworkErrorMapper
) : RoomsRepository {
    override suspend fun getRoomsList(
        page: Int?, size: Int
    ) = networkErrorMapper.safeApiCallMapped { roomApi.rooms(page, size) }

    override fun getRoomsPaged(): Flow<PagingData<Room>> = Pager(
        config = PagingConfig(
            pageSize = 100, prefetchDistance = 20, initialLoadSize = 100
        ), pagingSourceFactory = {
            RoomsPagingSource(this)
        }).flow

    override suspend fun createRoom(bio: String, displayName: String): Either<AppError, String> =
        networkErrorMapper.safeApiCall { roomApi.createRoom(CreateRoomDto(bio, displayName)) }
            .map { it.id }

    override suspend fun joinRoom(token: String): Either<AppError, String> =
        networkErrorMapper.safeApiCall { roomApi.invite(token) }
            .map { it.id }

    override suspend fun getRoomMembers(
        roomId: String,
        page: Int?,
        size: Int
    ): Either<AppError, PagingPage<Member>> =
        networkErrorMapper.safeApiCallMapped { roomApi.members(roomId, page, size) }

    override fun getMembersPaged(roomId: String): Flow<PagingData<Member>> = Pager(
        config = PagingConfig(
            pageSize = 100, prefetchDistance = 20, initialLoadSize = 100
        ), pagingSourceFactory = {
            MembersPagingSource(this, roomId)
        }).flow

    override suspend fun getRoomProfile(roomId: String): Either<AppError, Room> =
        networkErrorMapper.safeApiCallMapped { roomApi.room(roomId) }

    override suspend fun editRoomProfile(
        roomId: String,
        displayName: String?,
        bio: String?
    ): Either<AppError, Room> =
        networkErrorMapper.safeApiCallMapped {
            roomApi.editRoom(roomId, UpdateRoomDto(bio = bio, displayName = displayName))
        }

    override suspend fun updateRoomTimings(
        roomId: String,
        expiredNotify: Long?,
        reviewInterval: Long?
    ): Either<AppError, Room> =
        networkErrorMapper.safeApiCallMapped {
            roomApi.updateTimings(
                roomId,
                UpdateRoomTimingsDto(expiredNotify = expiredNotify, reviewInterval = reviewInterval)
            )
        }

    override suspend fun getInviteToken(roomId: String): Either<AppError, String> =
        networkErrorMapper.safeApiCall { roomApi.getInviteToken(roomId) }
            .map { it.token }


    override suspend fun deleteRoom(roomId: String): Either<AppError, Unit> =
        networkErrorMapper.safeApiCall { roomApi.deleteRoom(roomId) }
}