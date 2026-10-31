package com.example.zapas.core.network

import kotlinx.coroutines.CancellationException
import arrow.core.Either
import arrow.core.left
import arrow.core.right
import com.example.zapas.core.network.dto.AppErrJson
import com.example.zapas.core.network.dto.ResponseDto
import com.example.zapas.core.domain.model.AppError
import com.squareup.moshi.Moshi
import retrofit2.Response
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class NetworkErrorMapper @Inject constructor(
    val moshi: Moshi
) {
    inline fun <reified T> toEither(response: Response<T>): Either<AppError, T> {
        return if (response.isSuccessful) {
            val body = response.body()
            body?.right() ?: if (Unit is T) {
                @Suppress("UNCHECKED_CAST") (Unit as T).right()
            } else {
                AppError.UnknownError("Empty response body").left()
            }
        } else {
            val errorBody = response.errorBody()?.string()
            val appError = try {
                val adapter = moshi.adapter(AppErrJson::class.java)
                val errorJson = adapter.fromJson(errorBody ?: "")
                if (errorJson != null) {
                    AppError.ApiError(errorJson.code, errorJson.msg, response.code())
                } else {
                    AppError.ApiError(response.code(), response.message())
                }
            } catch (_: Exception) {
                AppError.ApiError(response.code(), response.message())
            }
            appError.left()
        }
    }

    suspend inline fun <reified T> safeApiCall(crossinline call: suspend () -> Response<T>): Either<AppError, T> {
        return try {
            toEither(call())
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            AppError.NetworkError(e).left()
        }
    }

    // Version for ResponseDto mapping
    suspend inline fun <reified DTO : ResponseDto<DOMAIN>, DOMAIN> safeApiCallMapped(
        crossinline call: suspend () -> Response<DTO>
    ): Either<AppError, DOMAIN> {
        return safeApiCall(call).map { it.toDomain() }
    }
}
