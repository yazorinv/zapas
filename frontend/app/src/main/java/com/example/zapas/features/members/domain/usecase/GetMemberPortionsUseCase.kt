package com.example.zapas.features.members.domain.usecase

import arrow.core.Either
import com.example.zapas.core.domain.UseCase
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.members.domain.model.MemberPortion
import com.example.zapas.features.members.domain.repository.MembersRepository
import javax.inject.Inject

class GetMemberPortionsUseCase @Inject constructor(
    private val membersRepository: MembersRepository
) : UseCase<GetMemberPortionsUseCase.Params, Either<AppError, List<MemberPortion>>>() {
    data class Params(val memberId: String, val begin: String, val end: String)

    override suspend fun invoke(params: Params) =
        membersRepository.getPortions(params.memberId, params.begin, params.end)
}
