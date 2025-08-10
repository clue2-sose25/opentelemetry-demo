package com.opentelemetry.demo.quote.controller;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.opentelemetry.demo.quote.dto.QuoteRequest;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.autoconfigure.web.servlet.WebMvcTest;
import org.springframework.boot.test.mock.mockito.MockBean;
import org.springframework.http.MediaType;
import org.springframework.test.web.servlet.MockMvc;
import com.opentelemetry.demo.quote.service.QuoteCalculationService;

import java.math.BigDecimal;

import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.Mockito.when;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.*;

@WebMvcTest(QuoteController.class)
class QuoteControllerTest {

    @Autowired
    private MockMvc mockMvc;

    @Autowired
    private ObjectMapper objectMapper;

    @MockBean
    private QuoteCalculationService quoteCalculationService;

    @Test
    void testGetQuote_ValidRequest() throws Exception {
        QuoteRequest request = new QuoteRequest(3);
        when(quoteCalculationService.calculateQuote(3)).thenReturn(new BigDecimal("87.30"));
        
        mockMvc.perform(post("/getquote")
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request)))
                .andExpect(status().isOk())
                .andExpect(content().contentType(MediaType.APPLICATION_JSON))
                .andExpect(jsonPath("$").value(87.30));
    }

    @Test
    void testGetQuote_InvalidRequest() throws Exception {
        QuoteRequest request = new QuoteRequest(0);
        when(quoteCalculationService.calculateQuote(0)).thenThrow(new IllegalArgumentException("Number of items must be greater than 0"));
        
        mockMvc.perform(post("/getquote")
                .contentType(MediaType.APPLICATION_JSON)
                .content(objectMapper.writeValueAsString(request)))
                .andExpect(status().isBadRequest());
    }

    @Test
    void testHealthEndpoint() throws Exception {
        mockMvc.perform(get("/health"))
                .andExpect(status().isOk())
                .andExpect(content().string("OK"));
    }
}
