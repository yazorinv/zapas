package com.example.zapas.features.rooms.domain.usecase

import arrow.core.Either
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.rooms.domain.model.Room
import com.example.zapas.features.rooms.domain.repository.RoomsRepository
import com.example.zapas.core.domain.UseCase
import javax.inject.Inject

class UpdateRoomTimingsUseCase @Inject constructor(
    private val roomsRepository: RoomsRepository
) : UseCase<UpdateRoomTimingsUseCase.Params, Either<AppError, Room>>() {

    data class Params(
        val roomId: String,
        val expiredNotify: Long?,
        val reviewInterval: Long?
    )

    override suspend fun invoke(params: Params): Either<AppError, Room> = roomsRepository.updateRoomTimings(
        roomId = params.roomId,
        expiredNotify = params.expiredNotify,
        reviewInterval = params.reviewInterval
    )
}
