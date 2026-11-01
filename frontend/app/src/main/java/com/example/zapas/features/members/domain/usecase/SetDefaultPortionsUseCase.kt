package com.example.zapas.features.members.domain.usecase

import arrow.core.Either
import com.example.zapas.core.domain.UseCase
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.members.domain.repository.MembersRepository
import javax.inject.Inject

class SetDefaultPortionsUseCase @Inject constructor(
    private val membersRepository: MembersRepository
) : UseCase<SetDefaultPortionsUseCase.Params, Either<AppError, Long>>() {
    data class Params(val memberId: String, val portions: Long)

    override suspend fun invoke(params: Params) =
        membersRepository.setDefaultPortions(params.memberId, params.portions)
}
