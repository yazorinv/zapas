package com.example.zapas.features.members.domain.model

/** A server-created daily record. date uses the ISO yyyy-MM-dd format. */
data class MemberPortion(val date: String, val portions: Long)
