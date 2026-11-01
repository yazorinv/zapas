package com.example.zapas.features.members.data.dto

import com.example.zapas.core.network.dto.ResponseDto
import com.example.zapas.features.members.domain.model.MemberPortion
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class MemberPortionDto(val date: String, val portions: Long) : ResponseDto<MemberPortion> {
    override fun toDomain() = MemberPortion(date, portions)
}
