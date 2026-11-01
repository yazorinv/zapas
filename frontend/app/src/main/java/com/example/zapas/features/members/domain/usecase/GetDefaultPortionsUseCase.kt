package com.example.zapas.features.members.domain.usecase

import arrow.core.Either
import com.example.zapas.core.domain.UseCase
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.members.domain.repository.MembersRepository

import javax.inject.Inject

class GetDefaultPortionsUseCase @Inject constructor(
    private val membersRepository: MembersRepository
) : UseCase<String, Either<AppError, Long>>() {
    override suspend fun invoke(params: String) = membersRepository.getDefaultPortions(params)
}
