package com.example.zapas.core.di

import com.example.zapas.features.members.data.api.MembersApiService
import com.example.zapas.features.members.data.repository.MembersRepositoryImpl
import com.example.zapas.features.members.domain.repository.MembersRepository
import android.content.Context
import com.example.zapas.features.auth.data.api.AuthApiService
import com.example.zapas.core.network.AuthInterceptor
import com.example.zapas.features.rooms.data.api.RoomsApiService
import com.example.zapas.features.profile.data.api.UserApiService
import com.example.zapas.core.network.NetworkErrorMapper
import com.example.zapas.core.local.TokenManager
import com.example.zapas.core.local.storage.KeyValueStorage
import com.example.zapas.core.local.storage.SharedPreferenceStorage
import com.example.zapas.core.network.TokenAuthenticator
import com.example.zapas.features.auth.data.repository.AuthRepositoryImpl
import com.example.zapas.features.rooms.data.repository.RoomsRepositoryImpl
import com.example.zapas.features.profile.data.repository.UserRepositoryImpl
import com.example.zapas.features.auth.domain.repository.AuthRepository
import com.example.zapas.features.rooms.domain.repository.RoomsRepository
import com.example.zapas.features.profile.domain.repository.UserRepository
import dagger.Lazy
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object AppModule {

    @Provides
    @Singleton
    fun provideMembersRepository(
        api: MembersApiService,
        errorMapper: NetworkErrorMapper
    ): MembersRepository =
        MembersRepositoryImpl(api, errorMapper)

    @Provides
    @Singleton
    fun provideKeyValueStorage(@ApplicationContext context: Context): KeyValueStorage =
        SharedPreferenceStorage(context)

    @Provides
    @Singleton
    fun provideTokenManager(storage: KeyValueStorage): TokenManager = TokenManager(storage)

    @Provides
    @Singleton
    fun provideAuthInterceptor(
        tokenManager: TokenManager
    ): AuthInterceptor = AuthInterceptor(tokenManager)

    @Provides @Singleton
    fun provideTokenAuthenticator(
        tokenManager: TokenManager,
        authRepository: Lazy<AuthRepository>
    ): TokenAuthenticator = TokenAuthenticator(tokenManager, authRepository)

    @Provides
    @Singleton
    fun provideAuthRepository(
        authApi: AuthApiService,
        tokenManager: TokenManager,
        networkErrorMapper: NetworkErrorMapper
    ): AuthRepository = AuthRepositoryImpl(authApi, tokenManager, networkErrorMapper)

    @Provides
    @Singleton
    fun provideUserRepository(
        userApi: UserApiService,
        tokenManager: TokenManager,
        networkErrorMapper: NetworkErrorMapper
    ): UserRepository = UserRepositoryImpl(userApi, tokenManager, networkErrorMapper)

    @Provides
    @Singleton
    fun provideRoomsRepository(
        roomsApi: RoomsApiService,
        networkErrorMapper: NetworkErrorMapper
    ): RoomsRepository = RoomsRepositoryImpl(roomsApi, networkErrorMapper)
}
