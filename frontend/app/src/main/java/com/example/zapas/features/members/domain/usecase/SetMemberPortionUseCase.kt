package com.example.zapas.features.members.domain.usecase

import arrow.core.Either
import com.example.zapas.core.domain.UseCase
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.members.domain.repository.MembersRepository
import javax.inject.Inject

class SetMemberPortionUseCase @Inject constructor(
    private val membersRepository: MembersRepository
) : UseCase<SetMemberPortionUseCase.Params, Either<AppError, Unit>>() {
    data class Params(val memberId: String, val date: String, val portions: Long)

    override suspend fun invoke(params: Params) =
        membersRepository.setPortion(params.memberId, params.date, params.portions)
}
