package com.example.zapas.features.members.data.api

import com.example.zapas.features.members.data.dto.DefaultPortionsDto
import com.example.zapas.features.members.data.dto.MemberPortionDto
import com.example.zapas.features.members.data.dto.OriginMemberDto
import com.example.zapas.features.members.data.dto.UpdateMemberProfileDto
import com.example.zapas.features.rooms.data.dto.response.MemberDto
import retrofit2.Response
import retrofit2.http.Body
import retrofit2.http.GET
import retrofit2.http.POST
import retrofit2.http.Path
import retrofit2.http.Query

interface MembersApiService {
    @GET("member/{member_id}")
    suspend fun member(@Path("member_id") memberId: String): Response<MemberDto>

    @GET("member/{member_id}/origin")
    suspend fun origin(@Path("member_id") memberId: String): Response<OriginMemberDto>

    @POST("member/{member_id}/profile")
    suspend fun updateProfile(
        @Path("member_id") memberId: String,
        @Body request: UpdateMemberProfileDto
    ): Response<OriginMemberDto>

    @GET("member/{member_id}/default_portions")
    suspend fun defaultPortions(@Path("member_id") memberId: String): Response<DefaultPortionsDto>

    @POST("member/{member_id}/default_portions")
    suspend fun setDefaultPortions(
        @Path("member_id") memberId: String,
        @Body request: DefaultPortionsDto
    ): Response<DefaultPortionsDto>

    // Requires the server update that moves the date range from a GET body to query parameters.
    @GET("member/{member_id}/portions")
    suspend fun portions(
        @Path("member_id") memberId: String,
        @Query("begin") begin: String,
        @Query("end") end: String
    ): Response<List<MemberPortionDto>>

    @POST("member/{member_id}/portion")
    suspend fun setPortion(
        @Path("member_id") memberId: String,
        @Body request: MemberPortionDto
    ): Response<Unit>
}
