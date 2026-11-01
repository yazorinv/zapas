package com.example.zapas.features.members.data.dto

import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class DefaultPortionsDto(val portions: Long)
