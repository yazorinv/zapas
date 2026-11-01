package com.example.zapas.features.members.data.dto.response

import com.example.zapas.core.network.dto.ResponseDto
import com.example.zapas.features.members.domain.model.MemberPortion
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class MemberPortionDto(
    @Json(name = "date") val date: String,
    @Json(name = "portions") val portions: Long
) : ResponseDto<MemberPortion> {
    override fun toDomain() = MemberPortion(date, portions)
}
