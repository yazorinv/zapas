package com.example.zapas.features.profile.data.repository

import arrow.core.Either
import com.example.zapas.core.network.dto.UuidDto
import com.example.zapas.core.network.NetworkErrorMapper
import com.example.zapas.core.local.TokenManager
import com.example.zapas.core.domain.model.AppError
import com.example.zapas.features.profile.domain.repository.EmailRepository
import com.example.zapas.features.profile.domain.repository.UserRepository
import com.example.zapas.features.profile.data.api.EmailApiService
import com.example.zapas.features.profile.data.api.UserApiService
import com.example.zapas.features.profile.data.dto.request.ChangePasswordRequestDto
import com.example.zapas.features.profile.data.dto.request.EmailDto
import com.example.zapas.features.profile.data.dto.request.UpdateProfileRequestDto
import javax.inject.Inject

class UserRepositoryImpl @Inject constructor(
    private val userApi: UserApiService,
    private val tokenManager: TokenManager,
    private val networkErrorMapper: NetworkErrorMapper
) : UserRepository {

    override suspend fun getMe() = networkErrorMapper.safeApiCallMapped { userApi.me() }

    override suspend fun updateProfile(
        displayName: String?, bio: String?, username: String?
    ) = networkErrorMapper.safeApiCallMapped {
        userApi.updateProfile(
            UpdateProfileRequestDto(
                displayName = displayName, bio = bio, username = username
            )
        )
    }

    override suspend fun changePassword(oldPasswd: String, newPasswd: String) =
        networkErrorMapper.safeApiCall {
            userApi.updatePassword(ChangePasswordRequestDto(oldPasswd, newPasswd))
        }.map { tokenManager.clearTokens() }

    override suspend fun deleteUser(userId: String) = networkErrorMapper.safeApiCall {
        userApi.deleteUser(UuidDto(userId))
    }

    override suspend fun logout() {
        try {
            userApi.logout()
        } catch (e: Exception) {
            e.printStackTrace()
        }
        tokenManager.clearTokens()
    }
}

class EmailRepositoryImpl @Inject constructor(
    private val emailApi: EmailApiService,
    private val networkErrorMapper: NetworkErrorMapper
) : EmailRepository {
    override suspend fun resetPassword(email: String): Either<AppError, Unit> = networkErrorMapper.safeApiCall {
        emailApi.forgotPassword(EmailDto(email))
    }

    override suspend fun updateEmail(email: String): Either<AppError, Unit> = networkErrorMapper.safeApiCall {
        emailApi.updateEmail(EmailDto(email))
    }

    override suspend fun verifyEmail(): Either<AppError, Unit> = networkErrorMapper.safeApiCall {
        emailApi.verifyEmail()
    }
}