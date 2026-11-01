package com.example.zapas.features.members.domain.usecase

import arrow.core.Either
import com.example.zapas.core.domain.UseCase
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.members.domain.repository.MembersRepository
import com.example.zapas.features.members.domain.model.LocalMemberProfile
import javax.inject.Inject

class GetLocalMemberProfileUseCase @Inject constructor(
    private val membersRepository: MembersRepository
) : UseCase<String, Either<AppError, LocalMemberProfile>>() {
    override suspend fun invoke(params: String) = membersRepository.getLocalProfile(params)
}
