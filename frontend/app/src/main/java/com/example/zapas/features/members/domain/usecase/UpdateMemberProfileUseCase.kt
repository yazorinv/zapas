package com.example.zapas.features.members.domain.usecase

import arrow.core.Either
import com.example.zapas.core.domain.UseCase
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.members.domain.model.LocalMemberProfile
import com.example.zapas.features.members.domain.repository.MembersRepository
import javax.inject.Inject

class UpdateMemberProfileUseCase @Inject constructor(
    private val membersRepository: MembersRepository
) : UseCase<UpdateMemberProfileUseCase.Params, Either<AppError, LocalMemberProfile>>() {
    data class Params(val memberId: String, val displayName: String?, val bio: String?)

    override suspend fun invoke(params: Params) = membersRepository.updateProfile(
        params.memberId, params.displayName, params.bio
    )
}
