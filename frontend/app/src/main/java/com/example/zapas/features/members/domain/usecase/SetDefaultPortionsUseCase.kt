package com.example.zapas.features.members.domain.usecase

import arrow.core.Either
import arrow.core.left
import com.example.zapas.core.domain.UseCase
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.members.domain.repository.MembersRepository
import javax.inject.Inject

class SetDefaultPortionsUseCase @Inject constructor(
    private val membersRepository: MembersRepository
) : UseCase<SetDefaultPortionsUseCase.Params, Either<AppError, Long>>() {
    data class Params(val memberId: String, val portions: Long)

    override suspend fun invoke(params: Params): Either<AppError, Long> {
        if (params.portions < 0)
            return AppError.UnknownError("Количество порций не может быть отрицательным").left()
        return membersRepository.setDefaultPortions(params.memberId, params.portions)
    }
}
